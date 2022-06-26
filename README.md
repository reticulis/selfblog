# selfblog
### ***All things in this project is for learning purposes***
***Project isn't done!***

Create your own simple blog!

*Actually, `selfblog` works only on localhost!*

# Install
```shell
git clone https://github.com/reticulis/selfblog.git
cd selfblog
cargo install --path .
```

# Usage
```shell
# create required files
selfblog init
# start http server with your blog
selfblog start
# create a new draft post for your blog
selfblog new_post "Hello world!" "Description!" 
# after edit your post, just mark that for ready
selfblog ready
# publish your a new post!
selfblog publish         
```
