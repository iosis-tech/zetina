export default async function subscribeEvents(
  url: string,
  fetchparams: string,
  sseparams: string,
  callback: (data: any) => void,
) {
  const events = new EventSource(`${url}?${sseparams}`, {
    withCredentials: true,
  });

  const eventSource = new ReadableStream({
    start(controller) {
      events.onmessage = (ev) => {
        controller.enqueue(JSON.parse(ev.data));
      };
      events.onerror = () => {
        controller.close();
        events.close();
      };
    },
    cancel() {
      events.close();
    },
  });

  const eventSink = new WritableStream({
    write(data) {
      callback(data);
    },
  });

  const response = await fetch(`${url}?${fetchparams}`, {
    method: "GET",
    credentials: "same-origin",
  }).then((r) => r.json());

  callback(response);

  eventSource.pipeTo(eventSink);

  return events;
}
