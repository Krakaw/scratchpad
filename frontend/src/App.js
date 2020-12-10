import React from 'react'
import './App.css'
import Nav from './components/Nav'
import { Switch, Route, BrowserRouter as Router } from 'react-router-dom'
import Scratches from './pages/Scratches'
import Repos from './pages/Repos'
import NotFound from './pages/NotFound'

function App () {
  return (
        <div className="App">
            <Nav/>
            <Router>
                <Switch>
                    <Route exact path={'/'}>
                        <Scratches />
                    </Route>
                    <Route exact path={'/repos'}>
                        <Repos />
                    </Route>
                    <Route path="*">
                        <NotFound/>
                    </Route>
                </Switch>
            </Router>

        </div>
  )
}

export default App
