use tetra_core::frames;

// Timers as defined in Annex A.1 LLC timers
const T251_SENDER_RETRY_TIMER: u32 = frames!(4); // 4 signalling frames
const T252_ACK_WAITING_TIMER: u32 = frames!(9);
const T261_SETUP_WAITING_TIMER: u32 = frames!(4);
const T263_DISCONNECT_WAITING_TIMER: u32 = frames!(4);
const T265_RECONNECT_WAITING_TIMER: u32 = frames!(4);
const T271_RECEIVER_NOT_READY_FOR_TX_TIMER: u32 = frames!(36);
const T272_RECEIVER_NOT_READY_FOR_RX_TIMER: u32 = frames!(18);
