package multiplier;

import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;
import org.springframework.web.bind.annotation.ResponseBody;
import org.springframework.web.bind.annotation.RestController;

import java.util.concurrent.TimeUnit;

@RestController
public class MultiplierController {
	@RequestMapping(value = "/multiply", method = RequestMethod.POST)
	public @ResponseBody
	MultiplicationOutput multiply(@RequestBody MultiplicationInput in) throws InterruptedException {
		return new MultiplicationOutput(in.x * in.y);
	}
}

class MultiplicationOutput {
	long result;

	public MultiplicationOutput(long result) {
		this.result = result;
	}

	public long getResult() {
		return result;
	}
}

class MultiplicationInput {
	long x, y;

	public long getX() {
		return x;
	}

	public long getY() {
		return y;
	}
}