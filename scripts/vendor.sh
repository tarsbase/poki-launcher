cargo vendor
tar -cvf vendor.tar vendor/*
rm -r vendor
xz vendor.tar
