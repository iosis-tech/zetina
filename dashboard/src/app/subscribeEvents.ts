export default function subscribeEvents(
  url: string,
  params: string,
  callback: (data: any) => void,
) {
  const events = new EventSource(`${url}?${params}`, {
    withCredentials: false,
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

  eventSource.pipeTo(eventSink);

  return events;
}
