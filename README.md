# Project 4: Reliable Transport Protocol 

## High-Level Approach
Our team set out to create a reliable way to send data over UDP, inspired by how well TCP works but made to handle unreliable networks better. We aimed to make sure that all data gets from one point to another correctly, in the right order, and without missing any pieces, without using the automatic help that TCP offers. We started with a basic approach where we sent one piece of data at a time and waited for a response before sending the next. From there, we gradually added more complex features. These included the ability to send multiple pieces of data at once, the option to send data again if it didn't arrive the first time, and ways to adjust our timing based on how fast or slow the network was responding.

## Challenges We Faced
1. **Learning About ACKs and Sequence Numbers:** Understanding how acknowledgments (ACKs) and sequence numbers work was key to dealing with repeated or misplaced packets. Figuring out how to use these tools to keep data correct and in order was our first big challenge.
2. **Figuring Out RTT and RTO:** We had to adjust the timeout for resending packets based on the Round-Trip Time (RTT) to deal with packets that come in the wrong order. This was tricky, especially when the network's conditions kept changing.
3. **Changing the Window Size:** We needed to change the size of the data window to match how fast or slow the internet was at any time. This required a good understanding of how to manage data flow and avoid network overload.
4. **Resending Packets:** We had to come up with a smart way to resend packets when they were lost or corrupted without using too much bandwidth or taking too much time.

## Key Features We Added
1. **Checking for Corruption with Hashing:** We used a hashing function at the receiving end to make sure packets were intact, especially when they didn't arrive in order. This way, we could check that the data was still correct before using it.
2. **Adjusting Window Size and RTT:** We followed the project's guidelines closely, using math and principles from our class to change the data window size and RTT calculations, making the data transfer more reliable and efficient.
3. **Handshake for Starting Communication:** Like TCP, we added a simple handshake process to get things set up before sending data, which helped start the data exchange smoothly.
4. **Using Reno for Congestion:** We chose the Reno strategy for managing how much data we send to avoid overwhelming the network and to keep data moving efficiently, even when the network was busy.

## How We Tested Our Work
To test our system, we used a bunch of test settings provided in the Python testing environment. These tests mimicked different network problems like lost packets, repeated packets, delays, and limits on how much data could be sent. By testing over and over and fixing issues as we found them, we made sure our system worked well in all sorts of situations.

## Wrapping Up
This project taught us a lot about how network protocols work and the challenges of sending data reliably over unreliable connections. By solving each problem step by step and testing thoroughly, we created a system that's both strong and efficient. We think the features and methods we used are a great base for a reliable way to send data across unpredictable networks.

