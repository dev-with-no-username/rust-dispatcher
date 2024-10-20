# rust-dispatcher

Dispatcher in rust, based on [Handling 1 Million Requests per Minute with Go](http://marcio.io/2015/07/handling-1-million-requests-per-minute-with-golang/).

To execute the app for see the results, is sufficient to do the following command:

```bash
task run
```

The performance difference between the two approaches in this dispatcher will depend on the specific workload and system environment.
Let's break down the two approaches so that we can think about their potential trade-offs:

## 1. **Approach 1: Send Messages to Each Receiver One After the Other**

- **Explanation**: You send 100 messages to **Receiver 1**, then 100 messages to **Receiver 2**, and so on, for all 10 receivers.
In this approach, each receiver will receive 100 messages in one burst, and then they will close.

- **Characteristics**:
  - Each receiver gets a "batch" of messages in a short period.
  - After sending all 100 messages to one receiver, the sender can move on to the next receiver.
  - The receivers will close in a sequential order, one after the other, as they finish receiving all of their messages.

- **Pros**:
  - Potentially better cache locality and fewer context switches, as you're focusing on a single channel at a time.
  - Might be more efficient if the receivers process messages in bursts (since they don't need to switch between different receivers).

- **Cons**:
  - Receivers are idle until all the messages for their channel have been sent.
  - Might not take full advantage of concurrent processing across multiple receivers.

## 2. **Approach 2: Send Messages Round-Robin (One Message to Each Receiver at a Time)**

- **Explanation**: You send one message to **Receiver 1**, one message to **Receiver 2**, and continue until you have sent one message to all 10 receivers. Then, you go back and repeat the process until all receivers have received 100 messages.

- **Characteristics**:
  - You distribute the messages evenly across all receivers in a round-robin fashion.
  - Each receiver gets its 100 messages slowly, one at a time, but all receivers get messages "concurrently" in the sense that none is waiting until all 100 messages for other receivers are sent.

- **Pros**:
  - More evenly distributed load across receivers, which could take advantage of concurrency if the receivers are being processed in parallel.
  - Each receiver starts receiving messages earlier and can start processing before all messages are sent.

- **Cons**:
  - Potentially more context switches between receivers, which could be less efficient for cache locality and messaging throughput.
  - If there are any delays or overhead from switching between different receivers, this might add some latency.

## Which Is Faster?

### 1. **Scenario 1 (Batching 100 messages at a time to each receiver)** might be faster when

- The system has low overhead when sending batches of messages to a single receiver.
- The receivers process messages efficiently in bursts, meaning processing many messages at once is better than processing one message at a time.
- You have minimal switching between senders and receivers, reducing context-switching overhead.

### 2. **Scenario 2 (Round-robin distribution of messages)** might be faster when

- Receivers can process messages concurrently, and you want to balance the load between them.
- The receivers start processing as soon as they receive a message, meaning processing can overlap, and all receivers start working earlier.
- The overhead of context-switching between receivers isn't significant.

## Which One Should You Choose?

- **If your receivers are I/O-bound (e.g., waiting on network, file, or database I/O)**: You may benefit from the **round-robin (Scenario 2)** approach because it allows all receivers to start processing sooner, and they can overlap their work while waiting on I/O.

- **If your receivers are CPU-bound and can process batches of messages efficiently**: The **batching approach (Scenario 1)** might be better because it allows each receiver to get all its work at once, reducing context switching and improving cache locality.
