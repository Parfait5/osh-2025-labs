# 实验二

进入shell目录后运行cargo run
## 一：目录导航

pwd直接打印，cd支持cd -或者没有第二段，也支持含~的路径

建立了简单的框架进行测试
### 选做题目

(1)cd 在没有第二个参数时，默认进入家目录

(2)cd - 可以切换为上一次所在的目录

## 二：管道

将指令拆分成|隔出的单条，再一条一条执行，前面的输出作为后面的输入
## 三：重定向

定义闭包 redirect 实现
## 四：信号处理

捕捉到 SIGINT 时，调用 handle_sigint 函数来处理
如果正在输入命令，就打印一个新的提示符;正在执行命令，父进程（Shell）不处理。

按下 Ctrl+C，操作系统会把 SIGINT 信号发送给整个前台进程组。子进程自动终止。


## 五：前后台进程

使用spawn()，立即启动进程，但不会等待它结束。
### 选做题目

(1)实现 fg 和 bg 命令
## 六：「可选」拓展功能

main中的loop输入收到EOF就退出
### 选做题目

(1)处理 CTRL-D (EOF) 按键
