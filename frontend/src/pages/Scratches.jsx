import React, { useEffect, useState } from 'react'
import { getInstances } from '../api'
import ScratchRow from '../components/ScratchRow'
function Scratches () {
  const [instances, setInstances] = useState([])
  const [api, setApi] = useState([])
  useEffect(() => {
    getInstances().then(i => {
      const sortedApi = i.api.map(i => {
        i.key = `${i.local} - ${i.remote}`
        // i.createdAtFrom = !i.exists ? 'Not deployed' : i.createdAt ? moment().to(i.createdAt) : '¯\\_(ツ)_/¯'
        // i.createdAt = moment(i.createdAt).format('YYYY-MM-DD HH:mm')
        i.extra = {
          labels: [],
          urls: [],
          ...i.extra
        }
        return i
      })
      sortedApi.sort((a, b) => {
        const aExists = a.exists ? 1 : 0
        const bExists = b.exists ? 1 : 0
        if (aExists < bExists) return 1
        if (aExists > bExists) return -1

        return a.local < b.local ? -1 : 1
      })
      setInstances(i)
      setApi(sortedApi)
      console.log(sortedApi)
    })
  }, [])
  return (
      <div className="row">
        <div className="col-sm-12">
          <table className="table table-striped">
            <thead>
            <tr>
              <th>Status</th>
              {/* <th scope="col">API Branch</th> */}
              {/* <th scope="col">Links</th> */}
              {/* <th>Deployed</th> */}
              {/* <th scope="col">Web Branch</th> */}

              {/* <th scope="col">Created</th> */}
              {/* <th scope="col">&nbsp;</th> */}
            </tr>

            </thead>
            <tbody>
            {api.map((instance, i) => <ScratchRow key={i} instance={instance}/>)}
            </tbody>
          </table>
        </div>
  </div>
  )
}
export default Scratches
