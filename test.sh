#!/bin/bash
#
# Low-budget test against system implementations...

set -eou pipefail

cd "`dirname "$0"`"

make

temp=$(mktemp)
iterations=1000

binaries=(md5 sha1 sha256 sha512)
system_equivs=(md5sum sha1sum sha256sum sha512sum)

for i in ${!binaries[@]}; do
    binary=${binaries[i]}
    system_equiv=${system_equivs[i]}

    if [ ! -e "${binary}" ] || [ "`which $system_equiv`" = "" ]; then
        echo "Missing dependency.  Test can't continue!"
        exit
    fi

    echo "Testing ${binary} against ${system_equiv}"

    result=$(
        for i in `seq 0 $iterations`; do
            head -c $i /dev/urandom > "$temp"; ./${binary} "$temp" | awk '{print $2}'; ${system_equiv} "$temp";
        done | awk '{print $1}' | uniq -u
          )


    if [ "$result" = "" ]; then
        echo "passed"
    else
        echo "FAILED"
    fi
done

rm -f "$temp"
