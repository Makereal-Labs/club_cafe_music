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

type ListEntry = {
  title: string,
  url: string,
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
  const [now_playing, setNowPlaying] = useState<ListEntry | null>(null);
  const [recv, setRecv] = useState<Array<ListEntry>>([]);
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
            body["now_playing"] as ListEntry | undefined;
          const queue = body["queue"] as Array<ListEntry>;
          if (msg_now_playing !== undefined) {
            setNowPlaying(msg_now_playing!);
          } else {
            setNowPlaying(null);
          }
          setRecv(queue);
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
            <Box sx={{ flexGrow: 1 }} />
            <ThemeToggle value={theme} onClick={on_theme_toggle} />
          </Toolbar>
        </AppBar>
        <Container>
          <Toolbar />
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
              <ListItem
                secondaryAction={
                  <IconButton edge="end" aria-label="copy link"
                    onClick={() => {
                      copyToClipboard(now_playing!.url);
                    }}>
                    <Link />
                  </IconButton>
                }
              >
                <ListItemText>
                  {now_playing!.title}
                </ListItemText>
              </ListItem>
              <Divider />
            </> : null}
            <ListSubheader>Queue</ListSubheader>
            {recv.map(item =>
              <>
                <ListItem
                  secondaryAction={
                    <IconButton edge="end" aria-label="copy link"
                      onClick={() => {
                        copyToClipboard(item.url);
                      }}>
                      <Link />
                    </IconButton>
                  }
                >
                  <ListItemText>
                    {item.title}
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
