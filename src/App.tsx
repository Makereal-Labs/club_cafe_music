import { useCallback, useState } from 'react'
import { AppBar, Box, Button, Container, List, ListItem, ListItemText, Toolbar, Typography } from '@mui/material'
import { useSession } from './session.ts'
import Player from './Player.tsx'

function App() {
  const [recv, setRecv] = useState<string[]>([])
  const session = useSession(
    // on open
    useCallback(() => { }, []),
    // on error
    useCallback(() => { }, []),
    // on message
    useCallback((event) => {
      console.log(event.data)
      setRecv(recv.concat([event.data]))
    }, [recv]),
    // on close
    useCallback(() => { }, []),
  )

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
        <Button variant="contained" onClick={() => { session.send("OAO") }}>Send message</Button>
        <List>
          <ListItem>
            <ListItemText>OAO</ListItemText>
          </ListItem>
          {recv.map(item =>
            <ListItem>
              <ListItemText>
                {item}
              </ListItemText>
            </ListItem>
          )}
          <ListItem>
            <ListItemText>OuO</ListItemText>
          </ListItem>
        </List>
      </Container>
    </Box>
  )
}

export default App
