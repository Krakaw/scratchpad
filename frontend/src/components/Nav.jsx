import React from 'react'
import logo from '../assets/logo.svg'

export default class Nav extends React.Component {
  render () {
    return (
            <nav className="navbar navbar-dark bg-dark">
                <a className="navbar-brand" href="/">
                    <img src={logo} width="30" height="30"
                         className="d-inline-block align-top" alt="" />
                        {'Scratchpad'}
                </a>
                <button className="navbar-toggler" type="button" data-toggle="collapse"
                        data-target="#navbarNavAltMarkup"
                        aria-controls="navbarNavAltMarkup" aria-expanded="false" aria-label="Toggle navigation">
                    <span className="navbar-toggler-icon"/>
                </button>
                <div className="collapse navbar-collapse" id="navbarNavAltMarkup">
                    <div className="navbar-nav">
                        <a className="nav-item nav-link active" href="#">Home <span className="sr-only">(current)</span></a>
                        <a className="nav-item nav-link" href="#">Features</a>
                        <a className="nav-item nav-link" href="#">Pricing</a>
                        <a className="nav-item nav-link disabled" href="#">Disabled</a>
                    </div>
                </div>
            </nav>
    )
  }
}
