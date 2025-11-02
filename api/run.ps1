$env:OPENSSL_DIR = 'C:\OpenSSL-Win64'
$env:OPENSSL_LIB_DIR = 'C:\OpenSSL-Win64\lib'
$env:OPENSSL_INCLUDE_DIR = 'C:\OpenSSL-Win64\include'
cd C:\dev\EventHub\api
cargo run
pause
