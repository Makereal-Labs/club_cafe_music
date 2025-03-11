import { useCallback, useState } from 'react';
import {
  AppBar,
  Box,
  Container,
  Divider,
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

function App() {
  const [theme, setTheme] = useState(ThemeId.Dark);
  const [now_playing, setNowPlaying] = useState<{title: string}| null>(null);
  const [recv, setRecv] = useState<string[]>([]);
  const [yt_link, setYtLink] = useState("");
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
            body["now_playing"] as {title: string} | undefined;
          const queue = body["queue"] as Array<{title: string}>;
          if (msg_now_playing !== undefined) {
            setNowPlaying(msg_now_playing!);
          } else {
            setNowPlaying(null);
          }
          setRecv(queue.map(item => item["title"]));
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

  return (
    <ThemeProvider theme={get_theme(theme)}>
      <Box>
        <AppBar>
          <Toolbar>
            <Typography variant="h6">
              Makereal Labs café music player
            </Typography>
          </Toolbar>
        </AppBar>
        <Container>
          <Toolbar />
          <ThemeToggle value={theme} onClick={on_theme_toggle} />
          <Player />
          <form onSubmit={event => {event.preventDefault(); on_yt_submit();} }>
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
            {now_playing ? <>
              <ListItem>
                <ListItemText>
                  {now_playing!.title}
                </ListItemText>
              </ListItem>
              <Divider />
            </> : null}
            <ListSubheader>Queue</ListSubheader>
            {recv.map(item =>
              <>
                <ListItem>
                  <ListItemText>
                    {item}
                  </ListItemText>
                </ListItem>
                <Divider />
              </>,
            )}
          </List>
        </Container>
      </Box>
    </ThemeProvider>
  );
}

export default App;
