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
  Toolbar,
  Typography,
} from '@mui/material';
import { useSession } from './session.ts';
import Player from './Player.tsx';

function App() {
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
          const queue = body["queue"] as Array<{title: string}>;
          setRecv(queue.map(item => item["title"]));
        }
      } catch {
        setRecv(recv.concat([event.data]));
      }
    }, [recv]),
    // on close
    useCallback(() => { }, []),
  );

  function on_yt_submit() {
    setYtLink("");
    const msg = {
      msg: "yt",
      link: yt_link,
    };
    session.send(JSON.stringify(msg));
  }

  return (
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
  );
}

export default App;
