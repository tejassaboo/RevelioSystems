package adder;

import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;
import org.springframework.web.bind.annotation.ResponseBody;
import org.springframework.web.bind.annotation.RestController;

import java.util.concurrent.TimeUnit;

@RestController
public class AdderController {
	@RequestMapping(value = "/add", method = RequestMethod.POST)
	public @ResponseBody
	AdditionOutput add(@RequestBody AdditionInput in) throws InterruptedException {
		long time = System.currentTimeMillis();
		int delay = (int) (25 + 12 * Math.sin(2 * Math.PI / 60000 * time));
		TimeUnit.MILLISECONDS.sleep(delay);
		return new AdditionOutput(in.x + in.y);
	}
}

class AdditionOutput {
	long result;

	public AdditionOutput(long result) {
		this.result = result;
	}

	public long getResult() {
		return result;
	}
}

class AdditionInput {
	long x, y;

	public long getX() {
		return x;
	}

	public long getY() {
		return y;
	}
}