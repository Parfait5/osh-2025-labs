# 实验三

## 必做部分测试方式说明

进入server目录，运行cargo run即可打开服务器

## 必做部分函数说明

main函数绑定地址监听, 如果失败返回连接失败。然后轮询读入请求，为每个请求单独建立线程，用 handle_client 函数处理请求。

在 handle_client 函数中，将请求中的东西读到缓冲区，再检查请求是否合理。为了保证程序的健壮性，使用循环读入数据放进缓冲区，用来解决无法一次连续读入的问题。然后解析请求，判断是否符合标准。再规范化路径，处理可能存在的错误，最后向 stream 中写入 header 和内容。

注：现在的 main.rs 是做了选做部分之后的，如果想看必做部分的话可以看直到 "lab: 完成了必做部分" 的内容

## 选做部分函数说明

### 使用线程池机制

创建 ThreadPool 结构体，workers 来保存所有线程的句柄，sender 来在主线程中把任务发送出去。实现了 new 和 execute 方法，new 创建线程池，execute 来阻塞式等待任务派发。

### 使用缓存机制

缓存结构体使用缓存表，键是文件路径 String，值是文件内容 Vec<u8>。首先用 `#[derive(Clone)]` 实现了浅拷贝，多个线程可以共享同一个缓存对象。然后实现 new 和 get_or_load 两个方法。new 创建一个新的、空的缓存，并用 Arc<Mutex<>> 封装起来，以便在线程池中共享。get_or_load 先查缓存，找不到就读磁盘，并缓存下来。

### 缓存机制带来的性能提升

下面分别是没有缓存和使用缓存的测试结果

![没有缓存的测试结果](https://github.com/Parfait5/osh-2025-labs/blob/master/lab3/server/figs/NoCache.png)

![使用缓存的测试结果](https://github.com/Parfait5/osh-2025-labs/blob/master/lab3/server/figs/WithCache.png)

通过测试我们可以看出，性能有较大的提升

![测试分析](https://github.com/Parfait5/osh-2025-labs/blob/master/lab3/server/figs/Test.png)
