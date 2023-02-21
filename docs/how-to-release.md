# 如何发布dubbo-rust的新版本

## 前置条件

安装[cargo-release](https://crates.io/crates/cargo-release):
```shell
$ cargo install cargo-release
```

登录 crates.io

```shell
$ cargo login
```

## 如何发布一个新版本

1. 更新版本号：
    ```shell
    $ cargo release version minor
    $ cargo release version minor --execute
    ```

2. 提交
    ```shell
    $ cargo release commit
    $ cargo release commit --execute
    ```

3. 打tag：
    ```shell
    $ cargo release tag
    $ cargo release tag --execute
    ```

4. push tag：
    ```shell
    $ cargo release push
    $ cargo release push --execute
    ```

5. 在邮件列表发起投票

    例子：<https://lists.apache.org/thread/cb949l81yfgg0wddthp1ym2wtd2y5s7j>

6. 发布到 crates.io

    ```shell
    $ cargo release publish
    $ cargo release publish --execute
    ```
