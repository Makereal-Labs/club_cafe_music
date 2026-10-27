export function format_time(time: number): string {
  const second = Math.floor(time % 60).toString().padStart(2, "0");
  const minute = Math.floor(time / 60);
  if (minute >= 60) {
    const minute2 = Math.floor(minute % 60).toString().padStart(2, "0");
    const hour = Math.floor(minute / 60);
    return `${hour}:${minute2}:${second}`;
  } else {
    return `${minute}:${second}`;
  }
}

