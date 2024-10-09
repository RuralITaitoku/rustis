set -x

if [ "$1" = "run" ];then
    set +x
    echo ========================================================
	set -x
	sh  stop_rustis.sh
	nohup ./target/debug/rustis > log.log 2>&1 &
	echo "kill $!" > stop_rustis.sh
elif [ "$1" = "release" ]; then
    set +x
    echo ========================================================
	set -x
    cargo build --release
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
elif [ "$1" = "install" ]; then
	cp -r src templates body.sh Cargo.lock Cargo.toml /Users/yuruyuru/relax/body/rustis

elif [ "$1" = "commit" ]; then
    set +x
    MES=$(date +"%Y年%m月%d日%H時%M分")
    set -x
    echo $MES
    git commit -m "$MES"
else
    set +x
    echo ========================================================
	clear
	cargo build
fi
