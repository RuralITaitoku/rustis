set -x

if [ "$1" = "run" ];then
	shft 3
	cargo run /Volumes/SSD480G/eroanime  -f .mp4 -t _m.mp4
elif [ "$1" = "check" ]; then
    set +x
    echo ▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️ ruralisプロセスチェック ▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️
    set -x
    ps -ax | grep rustis
elif [ "$1" = "net" ]; then
    set +x
    echo ▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️ ネットワークチェック158 ▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️▪️
    set -x
    if [ "$OS" = "Darwin" ]; then
    	netstat -an | grep 8080
    else
    	netstat -an | grep 8080
    fi
else
    echo ========================================================
	cargo build
fi
