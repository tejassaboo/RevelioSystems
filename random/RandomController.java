package random;

import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;
import org.springframework.web.bind.annotation.RestController;

import java.util.Random;
import java.util.concurrent.TimeUnit;

@RestController
public class RandomController {
	@RequestMapping(value = "/rand", method = RequestMethod.GET)
	public RandomOutput rand() throws InterruptedException {
		int time = 30000;
		int delay = 10;
		if (System.currentTimeMillis() % (time*2) < time) {
			delay = 20;
		}
		TimeUnit.MILLISECONDS.sleep(delay);
		Random rand = new Random();
		return new RandomOutput(rand.nextLong());
	}
}

class RandomOutput {
	long result;

	public RandomOutput(long result) {
		this.result = result;
	}

	public long getResult() {
		return result;
	}
}