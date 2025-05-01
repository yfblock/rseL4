# r(ust)seL4

## 环境安装

运行测试需要补充完成的 libsel4 版本等信息，我们采用已经生成好的信息使用。

### 安装 libsel4 相关的库
```
mkdir -p build
wget -qO- https://github.com/reL4team2/rel4-kernel-autobuild/releases/download/rel4-nomcs-release-2025-01-19/reL4.tar.gz | gunzip | tar -xvf - -C build --strip-components 2
rm build/bin/*
```

### 安装 add-payload tools

```shell
cargo install --git https://github.com/seL4/rust-sel4 --rev 1cd063a0f69b2d2045bfa224a36c9341619f0e9b sel4-kernel-loader-add-payload
```

### 安装 python 库

```shell
pip install lark inflection
```

说明：
- `lark` 库进行 bf 文件的解析
- `inflection` 进行命名的规范，包含 `snake_case` 和 `camel` 互相转换。
