# Revelio Systems

Revelio Systems is a student startup sponsored by UT Austin's Inventors Program in partnership with Trend Micro.
<br>

Team: Tejas Saboo, Soham Roy, Akshay Mantri, Wentao Yang
<br>
Pitch: https://youtu.be/I46GmiyUfdE
<br>

Revelio Systems is a software company that provides performance metrics for cloud-based microservices. We created a lab environment with a microservice that has the basic functionality of addition, multiplication, and random number generation to test our software. This environment has three physical servers -- one with a gateway that terminates the TLS connection to the system and runs Grafana and InfluxDB, one running our gateway (Zuul), and another running the rest of the services including Eureka. We isolate Zuul to ensure that we only collect network traffic going to and from the gateway. We have a Python script running on the Zuul server that monitors the local network interface -- it filters out HTTP packets, determines the duration of each request, and sends that data directly to our collection application. The collection application is written in Rust -- it takes the data coming from the sniffer and formats it to put in the database (sensitive information can be redacted at this point).
<br>

Technologies: Java, Python, Rust, HTML, Scapy, Grafana, InfluxDB, Eureka

