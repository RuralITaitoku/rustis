set -x



if [ "$1" == "setup" ]; then
	echo "# rustis" >> README.md
	git init
	git add README.md
	git commit -m "first commit"
	git branch -M main
	git remote add origin git@github.com:RuralITaitoku/rustis.git
	git push -u origin main
elif [ "$1" == "pop" ]; then
	git clone git@github.com:RuralITaitoku/rustis.git
fi

