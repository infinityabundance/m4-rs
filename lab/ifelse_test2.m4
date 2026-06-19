dnl Test 1: does ifelse expand args during collection?
define(`i', `5') ifelse(i, `5', `match', `no-match')
