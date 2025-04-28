# r(ust)seL4


## 运行测试

运行测试需要补充完成的 libsel4 版本等信息，我们采用已经生成好的信息使用。

```
mkdir -p build
wget -qO- https://github.com/reL4team2/rel4-kernel-autobuild/releases/download/rel4-nomcs-release-2025-01-19/reL4.tar.gz | gunzip | tar -xvf - -C build --strip-components 2
rm build/bin/*
```
