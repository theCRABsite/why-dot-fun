document.addEventListener("DOMContentLoaded", () => {
	const callButton = document.getElementById("callButton");
	const hangupButton = document.getElementById("hangupButton");
	const callTimer = document.getElementById("callTimer");
	const payButton = document.getElementById("payButton");

	let device;
	let callStartTime;
	let timerInterval;

	async function fetchTwilioToken(authToken) {
		const response = await fetch('https://gamecall-jvp99.ondigitalocean.app/twilio-token', {
			method: 'POST',
			headers: {
				'Authorization': `Bearer ${authToken}`
			}
		});

		if (!response.ok) {
			throw new Error('Failed to fetch Twilio token');
		}
		return response.text();
	}

	async function initializeTwilio(authToken) {
		try {
			const token = await fetchTwilioToken(authToken);
			device = new Twilio.Device(token);


		} catch (error) {
			console.error('Error initializing Twilio:', error);
		}
	}

	function toggleButtons(isCalling) {
		if (isCalling) {
			callButton.style.display = "none";
			hangupButton.style.display = "flex";
		} else {
			callButton.style.display = "flex";
			hangupButton.style.display = "none";
		}
	}

	function startTimer() {
		callStartTime = Date.now();
		callTimer.style.display = "block";

		timerInterval = setInterval(() => {
			const elapsedTime = Date.now() - callStartTime;
			const minutes = String(Math.floor(elapsedTime / 60000)).padStart(2, '0');
			const seconds = String(Math.floor((elapsedTime % 60000) / 1000)).padStart(2, '0');
			callTimer.textContent = `${minutes}:${seconds}`;
		}, 100);
	}

	function stopTimer() {
		clearInterval(timerInterval);
		callTimer.style.display = "none";
		callTimer.textContent = "";
	}

	async function startCall() {
		if (device) {
			const connection = await device.connect();

			connection.on('error', (error) => {
				console.error('Twilio Device error:', error.message);
				endCall();
			});

			connection.on('disconnect', () => {
				console.log('Call disconnected.');
				endCall();
			});

			if (connection) {
				toggleButtons(true);
				startTimer();
				console.log('Call started.');
			}
		} else {
			alert('Please pay to make a call.');
		}
	}

	function endCall() {
		if (device) {
			device.disconnectAll();
			toggleButtons(false);
			stopTimer();
		}
	}

	callButton.addEventListener("click", startCall);
	hangupButton.addEventListener("click", endCall);
	payButton.addEventListener("click", initializeTwilio);
});
