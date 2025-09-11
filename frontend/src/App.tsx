import { useCallback, useState } from 'react';
import {
  AppBar,
  Box,
  Container,
  Divider,
  IconButton,
  List,
  ListItem,
  ListItemText,
  ListSubheader,
  TextField,
  ThemeProvider,
  Toolbar,
  Typography,
} from '@mui/material';
import { useSession } from './session.ts';
import Player from './Player.tsx';
import ThemeToggle from './ThemeToggle.tsx';
import { get_theme, ThemeId } from './theme.ts';
import { Link } from '@mui/icons-material';
import CustomSnackbar from './CustomSnackbar.tsx';
import ChangelogView from './ChangelogView.tsx';

type ListEntry = {
  fetched: boolean,
  title: string,
  url: string,
  time: number,
};

function copyToClipboard(textToCopy: string) {
  // Navigator clipboard api needs a secure context (https)
  if (navigator.clipboard && window.isSecureContext) {
    navigator.clipboard.writeText(textToCopy);
  } else {
    // Use the 'out of viewport hidden text area' trick
    const textArea = document.createElement("textarea");
    textArea.value = textToCopy;

    // Move textarea out of the viewport so it's not visible
    textArea.style.position = "absolute";
    textArea.style.left = "-999999px";

    document.body.prepend(textArea);
    textArea.select();

    try {
      document.execCommand('copy');
    } catch (error) {
      console.error(error);
    } finally {
      textArea.remove();
    }
  }
}

function App() {
  const [theme, setTheme] = useState(ThemeId.Dark);
  // player
  const [playing, setPlaying] = useState(false);
  const [volume, setVolume] = useState(0);
  // queue
  const [now_playing, setNowPlaying] = useState<ListEntry | null>(null);
  const [recv, setRecv] = useState<Array<ListEntry>>([]);
  const [yt_link, setYtLink] = useState("");
  const [snackbar_message, setSnackbarMessage] =
    useState<string | undefined>(undefined);
  const [snackbar_key, setSnackbarKey] = useState(0);
  const session = useSession(
    // on open
    useCallback(() => { }, []),
    // on error
    useCallback(() => { }, []),
    // on message
    useCallback((event) => {
      try {
        const body = JSON.parse(event.data);
        if (body["msg"] == "queue") {
          const msg_now_playing =
            body["now_playing"] as ListEntry | undefined;
          const queue = body["queue"] as Array<ListEntry>;
          if (msg_now_playing !== undefined) {
            setNowPlaying(msg_now_playing!);
          } else {
            setNowPlaying(null);
          }
          setRecv(queue);
        } else if (body["msg"] == "player") {
          const playing = body["playing"] as boolean;
          const volume = body["volume"] as number;
          setPlaying(playing);
          setVolume(volume);
        } else if (body["msg"] == "snackbar") {
          const msg = body["text"] as string;
          display_snackbar(msg);
        }
      } catch {
        setRecv(recv.concat([event.data]));
      }
    }, [recv]),
    // on close
    useCallback(() => { }, []),
  );

  function on_theme_toggle() {
    if (theme == ThemeId.Light) {
      setTheme(ThemeId.Dark);
    } else {
      setTheme(ThemeId.Light);
    }
  }

  function on_yt_submit() {
    setYtLink("");
    const msg = {
      msg: "yt",
      link: yt_link,
    };
    session.send(JSON.stringify(msg));
  }

  function display_snackbar(message: string) {
    setSnackbarMessage(message);
    setSnackbarKey(new Date().getTime());
  }

  function on_player_button(action: string) {
    if (action == "pause") {
      setPlaying(false);
    } else if (action == "resume") {
      setPlaying(true);
    }
    const msg = {
      msg: "btn",
      action: action,
    };
    session.send(JSON.stringify(msg));
  }

  function on_volume_slider(volume: number) {
    setVolume(volume);
    const msg = {
      msg: "volume",
      volume: volume,
    };
    session.send(JSON.stringify(msg));
  }

  function gen_queue_entry(item: ListEntry) {
    let time = null;
    if (item.time) {
      const second = Math.floor(item.time % 60).toString().padStart(2, "0");
      const minute = Math.floor(item.time / 60);
      if (minute >= 60) {
        const minute2 = Math.floor(minute % 60).toString().padStart(2, "0");
        const hour = Math.floor(minute / 60);
        time = `${hour}:${minute2}:${second}`;
      } else {
        time = `${minute}:${second}`;
      }
    }

    return <>
      <ListItem
        secondaryAction={
          <IconButton edge="end" aria-label="copy link"
            onClick={() => {
              copyToClipboard(item.url);
              display_snackbar("Link copied!");
            }}>
            <Link />
          </IconButton>
        }
      >
        <ListItemText
          primary={item.fetched ? item.title : "Fetching..."}
          secondary={item.fetched ? time : (item.title ? item.title : item.url)}
        />
      </ListItem>
      <Divider />
    </>;
  }

  return (
    <ThemeProvider theme={get_theme(theme)}>
      <Box>
        <AppBar>
          <Toolbar>
            <Typography variant="h6">
              Makereal Labs café music player
            </Typography>
            <Box sx={{ flexGrow: 1 }} />
            <ThemeToggle value={theme} onClick={on_theme_toggle} />
            <ChangelogView />
          </Toolbar>
        </AppBar>
        <Container>
          <Toolbar />
          <Player
            playing={playing}
            volume={volume}
            onButton={on_player_button}
            onVolumeSlider={on_volume_slider}
          />
          <form onSubmit={event => { event.preventDefault(); on_yt_submit(); }}>
            <TextField
              fullWidth
              label="Youtube Link"
              type="search"
              variant="filled"
              autoComplete="off"
              margin="normal"
              value={yt_link}
              onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                setYtLink(event.target.value);
              }}
            />
          </form>
          <List>
            <ListSubheader>Now Playing</ListSubheader>
            {now_playing ? gen_queue_entry(now_playing) : null}
            <ListSubheader>Queue</ListSubheader>
            {recv.map(gen_queue_entry)}
          </List>
        </Container>
        <CustomSnackbar message={snackbar_message} key={snackbar_key} />
      </Box>
    </ThemeProvider>
  );
}

export default App;
