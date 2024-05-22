# GitHub OAuth Userinfo

This project is a proxy of `https://api.github.com/user`. 

当我们在自定义的SSO平台内使用GitHub作为身份提供者时，当我们使用api.github.com/user来获取用户信息的时候，对于某些用户，这个接口无法返回其邮件地址。本项目通过代理这个接口，当获取不到邮箱地址的时候，请求github的emails和public_emails接口来获取。

When using GitHub as an identity provider within our custom SSO platform, we have encountered an issue where the api.github.com/user endpoint fails to return the email address for certain users. This project addresses this issue by proxying the endpoint and, when an email address is not obtained, requests the user's email information from GitHub's emails and public_emails endpoints.


## 运行方式

### 构建镜像



```shell

docker build -t github-oauth-userinfo:latest .
```


### 运行

程序运行在10001端口上

```shell
docker run --rm github-oauth-userinfo:latest -p 10001:10001
```
