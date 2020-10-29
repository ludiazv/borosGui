// This file is generated with to embeed spec yml into static variable
pub const yml : &'static str = r#"
---
spec:
    - signature: { product: BM , model: 24M  , version: 4 }
      title: "Boros Met 24 with plain and mesh protocol v4"
      sections:
        - &GeneralMet
          name: General settings
          help: |
            This tab configures general behavior prameters:
              - Device ID: 16bit indentification of the sensor.
              - Enable led: Blink the led during notification. set it on for visual feedback.
              - Enable wake interrupt: Enable it if the sensor use external interrupt to start notification.
              - Notification interval: Notification period in 8s steps. e.g: 15 = 2 minutes
              - Payload template: template to build the payload (see docs for syntax)
          items:
            - Int:   { id: id  , caption: Device ID  , val: 1 , vmin: 0, vmax: 65536 }
            - Check: { id: led , caption: Enable led , val: false }
            - Check: { id: enint, caption: Enable wake interrupt, val: false }
            - Int:   { id: repo, caption: Notification interval, val: 15, vmin: 0, vmax: 65536 }
            - Text:  { id: tpl, caption: Payload template, val: "%Id,%Td,%Hd" , maxlen: 102 }
        - &RF24Config
          name: Radio configuration
          help: |
            This tab configure nrf24l01 configuration parameters:
              - RF24 mode: Plain or Mesh mode
              - Tx power: Radio output level (MIN,LOW,HIGH,MAX)
              - Enable Lna: Enable amplification if the module supports it.
              - Channel: radio chanel to use
              - Data Rate: 250Kbps, 1Mbps, 2Mbps
          items:
              - Choice: { id: mode, caption: RF24 mode , val: 0 , values: [ {val: 0 , desc: Plain }, { val : 1, desc: Mesh} ] }
              - Choice: { id: txp , caption: Tx power , val: 1 , values: [ {val: 0 , desc: Min }, {val: 1, desc: Low } , {val: 2, desc: High } , {val: 3, desc: Max } ] }
              - Check:  { id: lna , caption: Enable Lna, val: false }
              - Int:    { id: cha,  caption: Channel, val: 76, vmin: 1, vmax: 126 }
              - Choice: { id: rate, caption: Data rate, val: 1, values: [ {val: 0, desc: 250Kbps}, {val: 1 , desc: 1Mbps },{val: 2, desc: 2Mbps } ] }
        - &RF24Plain
          name: Plain mode
          help: |
            Parameters on this tab apply only if 'plain' mode is selected:
              - Pipe address size: Size of the destination pipe address.
              - Notification pipe: Hex address to notify sensors readings.
              - Payload size: Size of the fixed size payload. if 0 dynamic payload feature will be enabled.
              - CRC: Enale CRC check mode.
              - Enable Ack: Enable auto ack protocol of the nrf24l01 harware.
              - Retries: 0-15 retry attempts (if ack is enabled)
              - Retry delay: 0-15 Delay (n+1)*250us to wait before retry.
          items:
            - Int: { id: psz, caption: Pipe address size, val: 5, vmin: 3, vmax: 5}
            - Hex: { id: pipe, caption: Notificaiton pipe, val: AABBCCDDEE , maxlen: 5 , lsb: true }
            - Int: { id: dsz, caption: Payload size, val: 32 , vmin: 0, vmax: 32 }
            - Choice: { id: crc, caption: CRC, val: 2 , values: [ {val: 0, desc: Disabled }, {val: 1, desc: 8bit}, {val: 2, desc: 16bit} ] }
            - Check: { id: ack, caption: Enable Ack, val: true }
            - Int: { id: retr , caption: Retries, val: 8, vmin: 0, vmax: 15 }
            - Int: { id: retd , caption: Retry delay , val: 15, vmin: 0, vmax: 15 }
        - &RF24Mesh
          name: Mesh mode
          help: |
            Parameters on this tab apply only if 'mesh' mode is selected:
              - Node ID: Mesh node id of the sensor
              - Frame type: Mesh frame type id (byte)
              - Notification node ID: Node in the mesh to notify sensor information.
              - Force mesh renew: Force renew mesh network address in each notification.
          items:
              - Int: { id: mnid , caption: Node ID , val: 10, vmin: 1, vmax: 255 }
              - Int: { id: mfid , caption: Frame type , val: 30, vmin: 0, vmax: 255 }
              - Int: { id: mdst , caption: Notification node ID, val: 0, vmin: 0, vmax: 255 }
              - Check: { id: mfor, caption: Force mesh renew, val: false }
"#;
