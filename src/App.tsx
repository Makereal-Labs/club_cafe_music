import { useCallback, useState } from 'react'
import { AppBar, Box, Button, Container, List, ListItem, ListItemText, Toolbar, Typography } from '@mui/material'
import { useSession } from './session.ts'

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
        <Typography>
          Lorem ipsum dolor sit amet consectetur adipisicing elit. Similique unde
          fugit veniam eius, perspiciatis sunt? Corporis qui ducimus quibusdam,
          aliquam dolore excepturi quae. Distinctio enim at eligendi perferendis in
          cum quibusdam sed quae, accusantium et aperiam? Quod itaque exercitationem,
          at ab sequi qui modi delectus quia corrupti alias distinctio nostrum.
          Minima ex dolor modi inventore sapiente necessitatibus aliquam fuga et. Sed
          numquam quibusdam at officia sapiente porro maxime corrupti perspiciatis
          asperiores, exercitationem eius nostrum consequuntur iure aliquam itaque,
          assumenda et! Quibusdam temporibus beatae doloremque voluptatum doloribus
          soluta accusamus porro reprehenderit eos inventore facere, fugit, molestiae
          ab officiis illo voluptates recusandae.
        </Typography>
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
