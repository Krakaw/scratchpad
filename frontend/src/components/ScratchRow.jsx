import React from 'react'
function ScratchRow (instance) {
  return (
        <tr>
            <td>
                {JSON.stringify(instance, null, 2)}
            </td>
        </tr>
  )
}
export default ScratchRow
