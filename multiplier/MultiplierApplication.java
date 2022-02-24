package multiplier;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.cloud.client.discovery.EnableDiscoveryClient;

@SpringBootApplication
@EnableDiscoveryClient
public class MultiplierApplication {

	public static void main(String[] args) {
		SpringApplication.run(MultiplierApplication.class, args);
	}

}
