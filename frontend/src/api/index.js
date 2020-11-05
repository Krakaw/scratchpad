const serverUrl = process.env.REACT_APP_SERVER_URL || 'http://localhost:3000'

export const getInstances = async () => {
  return await (await fetch(`${serverUrl}/instances`)).json()
}
