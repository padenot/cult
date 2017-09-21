
extern crate libc;

use libc::{c_void, c_uint, c_int, c_schar, c_long};

pub const CUBEB_OK : c_int = 0;
pub const CUBEB_ERROR : c_int = -1;
pub const CUBEB_ERROR_INVALID_FORMAT : c_int = -2;
pub const CUBEB_ERROR_INVALID_PARAMETER : c_int = -3;
pub const CUBEB_ERROR_NOT_SUPPORTED : c_int = -4;
pub const CUBEB_ERROR_DEVICE_UNAVAILABLE : c_int = -5;

pub enum cubeb {}
pub enum cubeb_stream {}

pub type cubeb_sample_format = c_uint;

pub type cubeb_devid = *const c_void;

pub type cubeb_log_level = c_uint;

pub type cubeb_channel_layout = c_uint;

#[repr(C)]
#[derive(Debug)]
pub struct cubeb_stream_params {
    pub format: cubeb_sample_format,
    pub rate: u32,
    pub channels: u32,
    pub layout: cubeb_channel_layout,
}

#[repr(C)]
#[derive(Debug)]
pub struct cubeb_device {
    pub output_name: *mut c_schar,
    pub input_name: *mut c_schar,
}

pub type cubeb_state = c_int;

pub type cubeb_device_type = c_int;

pub type cubeb_device_state = c_int;

pub type cubeb_device_fmt = c_uint;

pub type cubeb_device_pref = c_uint;

#[repr(C)]
#[derive(Debug)]
pub struct cubeb_device_info {
    pub devid:          cubeb_devid,
    pub device_id:      *const c_schar,
    pub friendly_name:  *const c_schar,
    pub group_id:       *const c_schar,
    pub vendor_name:    *const c_schar,

    pub device_type:          cubeb_device_type,
    pub state:          cubeb_device_state,
    pub preferred:      cubeb_device_pref,

    pub format:         cubeb_device_fmt,
    pub default_format: cubeb_device_fmt,
    pub max_channels:   u32,
    pub default_rate:   u32,
    pub max_rate:       u32,
    pub min_rate:       u32,

    pub latency_lo:     u32,
    pub latency_hi:     u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct cubeb_device_collection {
    pub device: *const cubeb_device_info,
    pub count: usize,
}


pub type cubeb_data_callback = extern "C" fn (
    *const cubeb_stream, *mut c_void,
    *const c_void, *mut c_void, c_long) -> c_long;

pub type cubeb_state_callback = extern "C" fn (
    *const cubeb_stream, *mut c_void, cubeb_state);

pub type cubeb_device_changed_callback = extern "C" fn (*mut c_void);

pub type cubeb_device_collection_changed_callback = extern "C" fn (
    *const cubeb, *mut c_void);

//pub type cubeb_log_callback

#[link(name = "cubeb")]
extern {
    pub fn cubeb_init (context: *mut *const cubeb, context_name: *const c_schar,
                                             backend_name: *const c_schar) -> c_int;

    pub fn cubeb_get_backend_id(context: *const cubeb) -> *const c_schar;

    pub fn cubeb_get_max_channel_count(context: *const cubeb, max_channels: *mut u32)
            -> c_int;


    pub fn cubeb_get_min_latency(context: *const cubeb,
                                 params: *const cubeb_stream_params,
                                 latency_frames: *mut u32) -> c_int;

    pub fn cubeb_get_preferred_sample_rate(context: *const cubeb, rate: *mut u32)
            -> c_int;

    pub fn cubeb_get_preferred_channel_layout(context: *const cubeb, layout: *mut cubeb_channel_layout) -> c_int;

    pub fn cubeb_destroy(context: *const cubeb);

    pub fn cubeb_stream_init(context: *const cubeb,
                                stream: *mut *const cubeb_stream,
                                stream_name: *const c_schar,
                                input_device: cubeb_devid,
                                input_stream_params: *mut cubeb_stream_params,
                                output_device: cubeb_devid,
                                output_stream_params: *mut cubeb_stream_params,
                                latency_frames: u32,
                                data_callback: Option<cubeb_data_callback>,
                                state_callback: Option<cubeb_state_callback>,
                                user_ptr: *mut c_void) -> c_int;

    pub fn cubeb_stream_destroy(stream: *const cubeb_stream);

    pub fn cubeb_stream_start(stream: *const cubeb_stream) -> c_int;

    pub fn cubeb_stream_stop(stream: *const cubeb_stream) -> c_int;

    pub fn cubeb_stream_reset_default_device(stream: *const cubeb_stream) -> c_int;

    pub fn cubeb_stream_get_position(stream: *const cubeb_stream, position: *mut u64) -> c_int;

    pub fn cubeb_stream_get_latency(stream: *const cubeb_stream, latency: *mut u32) -> c_int;

    pub fn cubeb_stream_set_volume(stream: *const cubeb_stream, volume: f32) -> c_int;

    pub fn cubeb_stream_set_panning(stream: *const cubeb_stream, panning: f32) -> c_int;

    pub fn cubeb_stream_get_current_device(stream: *const cubeb_stream,
                                           device: *mut *const cubeb_device) -> c_int;

    pub fn cubeb_stream_device_destroy(stream: *const cubeb_stream,
                                       devices: *mut cubeb_device) -> c_int;

    pub fn cubeb_stream_register_device_changed_callback(stream: *const cubeb_stream,
                                                         callback: cubeb_device_changed_callback) -> c_int;

    pub fn cubeb_enumerate_devices(context: *const cubeb,
                                   devtype: cubeb_device_type,
                                   collection: *mut cubeb_device_collection) -> c_int;

    pub fn cubeb_device_collection_destroy(context: *const cubeb,
                                           collection: *mut cubeb_device_collection) -> c_int;

    pub fn cubeb_register_device_collection_changed(context: *const cubeb,
                                                    devtype: cubeb_device_type,
                                                    callback: cubeb_device_collection_changed_callback,
                                                    user_ptr: *mut c_void) -> c_int;

    // pub fn cubeb_set_log_callback(log_level: cubeb_log_level,
    //                               callback: cubeb_log_callback) -> c_int;
}
