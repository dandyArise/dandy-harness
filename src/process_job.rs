//! OS-owned cleanup of the engine's descendants, including on GUI crashes.
use std::{io, os::windows::io::AsRawHandle, process::Child};
use windows_sys::Win32::System::{
    Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    },
    Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    },
};

pub struct ProcessJob(HANDLE);
// The uniquely owned handle is accessed only through the engine mutex.
unsafe impl Send for ProcessJob {}

impl ProcessJob {
    pub fn attach(child: &Child) -> io::Result<Self> {
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = Self(handle);
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const _,
                std::mem::size_of_val(&limits) as u32,
            ) == 0
                || AssignProcessToJobObject(handle, child.as_raw_handle()) == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        }
    }

    pub fn stop(&self) {
        unsafe {
            TerminateJobObject(self.0, 1);
        }
    }

    /// The child is created suspended: it cannot spawn descendants before
    /// assignment. Resume only its primary thread after the job is attached.
    pub fn resume(child: &Child) -> io::Result<()> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }
            let mut entry: THREADENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of_val(&entry) as u32;
            let mut found = Thread32First(snapshot, &mut entry);
            let mut result = Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Primary engine thread not found",
            ));
            while found != 0 {
                if entry.th32OwnerProcessID == child.id() {
                    let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                    result = if thread.is_null() {
                        Err(io::Error::last_os_error())
                    } else {
                        let resumed = ResumeThread(thread);
                        let error = io::Error::last_os_error();
                        CloseHandle(thread);
                        if resumed == u32::MAX {
                            Err(error)
                        } else {
                            Ok(())
                        }
                    };
                    break;
                }
                found = Thread32Next(snapshot, &mut entry);
            }
            CloseHandle(snapshot);
            result
        }
    }
}

impl Drop for ProcessJob {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
