#!/bin/bash
# m4-rs QEMU VM setup for GNU m4 oracle testing
#
# This script creates a disposable Ubuntu LTS VM with GNU m4 and its
# full testsuite installed. GPL code stays ON THE VM — only receipts
# and comparison results are copied back to the m4-rs repo.
#
# Requirements: qemu-system-x86_64, qemu-img, wget, cloud-localds
#
# Usage: bash lab/corpus/setup-qemu-vm.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VM_DIR="$SCRIPT_DIR/vm"
mkdir -p "$VM_DIR"

VM_IMAGE="$VM_DIR/ubuntu-m4-oracle.qcow2"
VM_SIZE="10G"
UBUNTU_URL="https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.img"
SEED_IMAGE="$VM_DIR/seed.img"

# Step 1: Download Ubuntu cloud image if not present
if [ ! -f "$VM_IMAGE" ]; then
    echo "=== Downloading Ubuntu 24.04 LTS cloud image ==="
    wget -O "$VM_IMAGE.tmp" "$UBUNTU_URL"
    qemu-img resize "$VM_IMAGE.tmp" "$VM_SIZE"
    mv "$VM_IMAGE.tmp" "$VM_IMAGE"
    echo "Image downloaded and resized to $VM_SIZE"
fi

# Step 2: Create cloud-init seed ISO
cat > "$VM_DIR/user-data" << 'USERDATA'
#cloud-config
hostname: m4-oracle
manage_etc_hosts: true

users:
  - name: m4user
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    lock_passwd: true
    ssh_authorized_keys: []

packages:
  - m4
  - autoconf
  - automake
  - gcc
  - make
  - git
  - wget
  - diffutils
  - python3
  - bison
  - flex
  - texinfo

write_files:
  - path: /home/m4user/setup-gnu-m4.sh
    permissions: '0755'
    content: |
      #!/bin/bash
      set -e

      echo "=== Installing GNU m4 from source for full testsuite ==="

      # Download GNU m4 source (for testsuite only — NOT copied into m4-rs repo)
      cd /tmp
      wget -q https://ftp.gnu.org/gnu/m4/m4-1.4.19.tar.xz
      tar xf m4-1.4.19.tar.xz

      # Build GNU m4 with tests enabled
      cd m4-1.4.19
      ./configure --prefix=/usr/local/m4-oracle
      make -j$(nproc)
      sudo make install

      # Also build with changeword support for complete coverage
      ./configure --prefix=/usr/local/m4-oracle-changeword --enable-changeword
      make -j$(nproc)
      sudo make install

      # Run the testsuite to verify oracle
      echo "=== Running GNU m4 testsuite ==="
      make check || true  # Some tests may fail in minimal VM

      # Extract test inputs for oracle comparison
      echo "=== Extracting test inputs ==="
      mkdir -p /home/m4user/testsuite-extracts

      # Copy all .m4 test files
      find tests/ -name "*.m4" -exec cp {} /home/m4user/testsuite-extracts/ \;
      find tests/ -name "*.at" -exec cp {} /home/m4user/testsuite-extracts/ \;

      # Copy the test macros
      cp -r examples/ /home/m4user/testsuite-extracts/examples/

      echo "=== GNU m4 testsuite installed at /usr/local/m4-oracle/bin/m4 ==="
      echo "Test files extracted to /home/m4user/testsuite-extracts/"

  - path: /home/m4user/run-comparison.sh
    permissions: '0755'
    content: |
      #!/bin/bash
      # This script runs the comparison between GNU m4 and m4-rs.
      # It is run INSIDE the VM, with m4-rs compiled outside and scp'd in.
      set -e

      M4_ORACLE="/usr/local/m4-oracle/bin/m4"
      M4RS_BIN="/home/m4user/m4-rs"
      TEST_DIR="/home/m4user/testsuite-extracts"
      RESULTS="/home/m4user/comparison-results"

      mkdir -p "$RESULTS"

      echo "=== Oracle: $($M4_ORACLE --version | head -1) ==="
      echo "=== m4-rs:  $($M4RS_BIN --version 2>/dev/null || echo 'version: unknown') ==="

      # Run each .m4 test through both oracles
      PASS=0
      FAIL=0
      TOTAL=0

      for test_file in "$TEST_DIR"/*.m4; do
          if [ ! -f "$test_file" ]; then continue; fi
          TOTAL=$((TOTAL + 1))
          base=$(basename "$test_file")

          # Run oracle
          timeout 5 "$M4_ORACLE" "$test_file" > "$RESULTS/${base}.oracle.stdout" 2> "$RESULTS/${base}.oracle.stderr" || true
          echo $? > "$RESULTS/${base}.oracle.exit"

          # Run m4-rs
          timeout 5 "$M4RS_BIN" "$test_file" > "$RESULTS/${base}.rust.stdout" 2> "$RESULTS/${base}.rust.stderr" || true
          echo $? > "$RESULTS/${base}.rust.exit"

          # Compare stdout
          if diff -q "$RESULTS/${base}.oracle.stdout" "$RESULTS/${base}.rust.stdout" > /dev/null 2>&1; then
              PASS=$((PASS + 1))
              echo "PASS: $base"
          else
              FAIL=$((FAIL + 1))
              echo "FAIL: $base"
          fi
      done

      echo ""
      echo "=== Results: $PASS/$TOTAL passed, $FAIL failed ==="
      echo "$PASS" > "$RESULTS/summary.txt"
      echo "$FAIL" >> "$RESULTS/summary.txt"
      echo "$TOTAL" >> "$RESULTS/summary.txt"

  - path: /home/m4user/test-m4-rs.sh
    permissions: '0755'
    content: |
      #!/bin/bash
      # Quick smoke test after copying m4-rs binary
      M4RS="/home/m4user/m4-rs"

      echo "=== Testing m4-rs binary ==="
      echo "define(\`hello', \`world')hello" | "$M4RS"

      echo ""
      echo "=== Running comparison ==="
      bash /home/m4user/run-comparison.sh

runcmd:
  - echo "=== VM booted, running setup ==="
  - su - m4user -c 'bash /home/m4user/setup-gnu-m4.sh'
  - echo "=== Setup complete. VM ready for oracle testing. ==="
  - echo "=== To run comparison: scp m4-rs binary to VM, then: su - m4user -c 'bash /home/m4user/run-comparison.sh' ==="
USERDATA

cat > "$VM_DIR/meta-data" << 'METADATA'
instance-id: m4-oracle-001
local-hostname: m4-oracle
METADATA

# Create seed image
if command -v cloud-localds &> /dev/null; then
    cloud-localds "$SEED_IMAGE" "$VM_DIR/user-data" "$VM_DIR/meta-data"
else
    echo "WARNING: cloud-localds not found. Install cloud-utils package."
    echo "On Ubuntu: sudo apt-get install cloud-image-utils"
fi

echo ""
echo "=== QEMU VM setup complete ==="
echo ""
echo "To start the VM (no graphics):"
echo "  qemu-system-x86_64 \\"
echo "    -m 2048 \\"
echo "    -smp 2 \\"
echo "    -drive file=$VM_IMAGE,format=qcow2,if=virtio \\"
echo "    -drive file=$SEED_IMAGE,format=raw,if=virtio \\"
echo "    -nic user,hostfwd=tcp::2222-:22 \\"
echo "    -nographic"
echo ""
echo "After boot (~2 min), SSH in:"
echo "  ssh -p 2222 m4user@localhost"
echo ""
echo "Then copy m4-rs and run comparison:"
echo "  scp -P 2222 target/release/m4-rs m4user@localhost:~/"
echo "  ssh -p 2222 m4user@localhost 'bash /home/m4user/run-comparison.sh'"
echo ""
echo "Copy results back:"
echo "  scp -P 2222 m4user@localhost:~/comparison-results/* lab/corpus/receipts/"
