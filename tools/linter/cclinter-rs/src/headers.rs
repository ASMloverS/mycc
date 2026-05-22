use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

pub const C_SYS_HEADER: u8 = 1;
pub const CPP_SYS_HEADER: u8 = 2;
pub const OTHER_SYS_HEADER: u8 = 3;
pub const LIKELY_MY_HEADER: u8 = 4;
pub const POSSIBLE_MY_HEADER: u8 = 5;
pub const OTHER_HEADER: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncludeKind {
    Quoted, // "file.h"
    System, // <file.h>
}

pub static CPP_HEADERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "algorithm",
        "any",
        "array",
        "atomic",
        "barrier",
        "bit",
        "bitset",
        "charconv",
        "chrono",
        "codecvt",
        "compare",
        "complex",
        "concepts",
        "condition_variable",
        "coroutine",
        "deque",
        "exception",
        "execution",
        "expected",
        "filesystem",
        "format",
        "forward_list",
        "fstream",
        "functional",
        "future",
        "initializer_list",
        "iomanip",
        "ios",
        "iosfwd",
        "iostream",
        "iterator",
        "latch",
        "limits",
        "list",
        "locale",
        "map",
        "memory",
        "memory_resource",
        "mutex",
        "new",
        "numbers",
        "numeric",
        "optional",
        "ostream",
        "print",
        "queue",
        "random",
        "ranges",
        "ratio",
        "regex",
        "scoped_allocator",
        "semaphore",
        "set",
        "shared_mutex",
        "source_location",
        "span",
        "sstream",
        "stack",
        "stdexcept",
        "stop_token",
        "streambuf",
        "string",
        "string_view",
        "strstream",
        "syncstream",
        "system_error",
        "thread",
        "tuple",
        "type_traits",
        "typeindex",
        "typeinfo",
        "unordered_map",
        "unordered_set",
        "utility",
        "valarray",
        "variant",
        "vector",
        "version",
        "cassert",
        "cctype",
        "cerrno",
        "cfenv",
        "cfloat",
        "cinttypes",
        "ciso646",
        "climits",
        "clocale",
        "cmath",
        "csetjmp",
        "csignal",
        "cstdalign",
        "cstdarg",
        "cstdbool",
        "cstddef",
        "cstdint",
        "cstdio",
        "cstdlib",
        "cstring",
        "ctgmath",
        "ctime",
        "cuchar",
        "cwchar",
        "cwctype",
        "contract",
        "debugging",
        "hazard_pointer",
        "inplace_vector",
        "linalg",
        "mdspan",
        "parameter",
        "rcu",
        "spanstream",
        "stacktrace",
        "stdfloat",
        "text_encoding",
    ])
});

pub static C_HEADERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "assert.h",
        "complex.h",
        "ctype.h",
        "errno.h",
        "fenv.h",
        "float.h",
        "inttypes.h",
        "iso646.h",
        "limits.h",
        "locale.h",
        "math.h",
        "setjmp.h",
        "signal.h",
        "stdalign.h",
        "stdarg.h",
        "stdatomic.h",
        "stdbool.h",
        "stddef.h",
        "stdint.h",
        "stdio.h",
        "stdlib.h",
        "stdnoreturn.h",
        "string.h",
        "strings.h",
        "tgmath.h",
        "threads.h",
        "time.h",
        "uchar.h",
        "wchar.h",
        "wctype.h",
        "stdbit.h",
        "stdckdint.h",
        "dirent.h",
        "dlfcn.h",
        "fcntl.h",
        "fnmatch.h",
        "getopt.h",
        "glob.h",
        "grp.h",
        "pthread.h",
        "pwd.h",
        "regex.h",
        "sched.h",
        "semaphore.h",
        "spawn.h",
        "syslog.h",
        "termios.h",
        "unistd.h",
        "elf.h",
        "features.h",
        "malloc.h",
        "arpa/inet.h",
        "netdb.h",
        "netinet/in.h",
        "netinet/tcp.h",
        "sys/ipc.h",
        "sys/mman.h",
        "sys/msg.h",
        "sys/resource.h",
        "sys/select.h",
        "sys/sem.h",
        "sys/shm.h",
        "sys/socket.h",
        "sys/stat.h",
        "sys/statvfs.h",
        "sys/time.h",
        "sys/types.h",
        "sys/uio.h",
        "sys/un.h",
        "sys/utsname.h",
        "sys/wait.h",
        "aio.h",
        "alloca.h",
        "ar.h",
        "ctype.h",
        "db.h",
        "dir.h",
        "dlfcn.h",
        "endian.h",
        "envz.h",
        "err.h",
        "errno.h",
        "execinfo.h",
        "fcntl.h",
        "features.h",
        "fenv.h",
        "fmtmsg.h",
        "fnmatch.h",
        "ftw.h",
        "getopt.h",
        "glob.h",
        "grp.h",
        "iconv.h",
        "ifaddrs.h",
        "langinfo.h",
        "libgen.h",
        "libintl.h",
        "link.h",
        "locale.h",
        "malloc.h",
        "monetary.h",
        "mqueue.h",
        "nl_types.h",
        "nss.h",
        "paths.h",
        "poll.h",
        "printf.h",
        "procfs.h",
        "pthread.h",
        "pty.h",
        "pwd.h",
        "re_comp.h",
        "regex.h",
        "regexp.h",
        "resolv.h",
        "sched.h",
        "search.h",
        "semaphore.h",
        "shadow.h",
        "signal.h",
        "spawn.h",
        "stab.h",
        "stdatomic.h",
        "stdint.h",
        "stdio_ext.h",
        "strings.h",
        "stropts.h",
        "syslog.h",
        "termios.h",
        "tgmath.h",
        "threads.h",
        "ulimit.h",
        "unistd.h",
        "utime.h",
        "utmp.h",
        "utmpx.h",
        "values.h",
        "wait.h",
        "wchar.h",
        "wordexp.h",
        "a.out.h",
        "aliases.h",
        "alsa/asoundlib.h",
        "arpa/ftp.h",
        "arpa/nameser.h",
        "arpa/nameser_compat.h",
        "arpa/telnet.h",
        "arpa/tftp.h",
        "asm/byteorder.h",
        "asm/ioctls.h",
        "asm/page.h",
        "asm/posix_types.h",
        "asm/setup.h",
        "asm/sigcontext.h",
        "bits/byteswap.h",
        "bits/confname.h",
        "bits/dirent.h",
        "bits/elfclass.h",
        "bits/endian.h",
        "bits/environments.h",
        "bits/errno.h",
        "bits/fcntl.h",
        "bits/in.h",
        "bits/inf.h",
        "bits/ioctl-types.h",
        "bits/ioctls.h",
        "bits/ipctypes.h",
        "bits/locale.h",
        "bits/mathcalls.h",
        "bits/mman.h",
        "bits/nan.h",
        "bits/netdb.h",
        "bits/posix1_lim.h",
        "bits/posix2_lim.h",
        "bits/posix_opt.h",
        "bits/pthreadtypes.h",
        "bits/resource.h",
        "bits/sched.h",
        "bits/select.h",
        "bits/semaphore.h",
        "bits/setjmp.h",
        "bits/sigaction.h",
        "bits/sigcontext.h",
        "bits/siginfo.h",
        "bits/signum.h",
        "bits/sigset.h",
        "bits/sigstack.h",
        "bits/sockaddr.h",
        "bits/socket.h",
        "bits/socket_type.h",
        "bits/stat.h",
        "bits/statfs.h",
        "bits/statvfs.h",
        "bits/stdio_lim.h",
        "bits/time.h",
        "bits/timerfd.h",
        "bits/timex.h",
        "bits/types.h",
        "bits/typesizes.h",
        "bits/uio.h",
        "bits/utsname.h",
        "bits/waitflags.h",
        "bits/waitstatus.h",
        "bits/wordsize.h",
        "bits/xopen_lim.h",
        "drm/drm.h",
        "drm/drm_mode.h",
        "drm/drm_fourcc.h",
        "gnu/libc-version.h",
        "gnu/lib-names.h",
        "gnu/stubs.h",
        "linux/bpf.h",
        "linux/can.h",
        "linux/can/raw.h",
        "linux/filter.h",
        "linux/futex.h",
        "linux/if.h",
        "linux/if_addr.h",
        "linux/if_arp.h",
        "linux/if_ether.h",
        "linux/if_packet.h",
        "linux/if_tun.h",
        "linux/input.h",
        "linux/ioctl.h",
        "linux/kernel.h",
        "linux/limits.h",
        "linux/major.h",
        "linux/netlink.h",
        "linux/perf_event.h",
        "linux/posix_types.h",
        "linux/rtnetlink.h",
        "linux/sched.h",
        "linux/seccomp.h",
        "linux/sockios.h",
        "linux/types.h",
        "linux/videodev2.h",
        "misc/bcm2835.h",
        "mtd/mtd-abi.h",
        "mtd/mtd-user.h",
        "net/ethernet.h",
        "net/if.h",
        "net/if_arp.h",
        "net/if_packet.h",
        "net/ppp_defs.h",
        "net/route.h",
        "netinet/ether.h",
        "netinet/icmp6.h",
        "netinet/if_ether.h",
        "netinet/igmp.h",
        "netinet/in_systm.h",
        "netinet/ip.h",
        "netinet/ip6.h",
        "netinet/ip_icmp.h",
        "netinet/tcp.h",
        "netinet/udp.h",
        "protocols/routed.h",
        "protocols/talkd.h",
        "protocols/timed.h",
        "rdma/rdma_user_cm.h",
        "rdma/ib_user_verbs.h",
        "rpc/auth.h",
        "rpc/clnt.h",
        "rpc/pmap_clnt.h",
        "rpc/pmap_prot.h",
        "rpc/rpc.h",
        "rpc/rpc_msg.h",
        "rpc/svc.h",
        "rpc/svc_auth.h",
        "rpc/types.h",
        "rpc/xdr.h",
        "rpcsvc/bootparam.h",
        "rpcsvc/mount.h",
        "rpcsvc/nfs_prot.h",
        "rpcsvc/rquota.h",
        "rpcsvc/rusers.h",
        "rpcsvc/rwall.x",
        "rpcsvc/yp.h",
        "rpcsvc/ypclnt.h",
        "scsi/scsi.h",
        "scsi/sg.h",
        "sound/asound.h",
        "sound/compress_offload.h",
        "video/edid.h",
        "video/sisfb.h",
        "video/uvesafb.h",
        "xen/evtchn.h",
        "xen/gnttab.h",
        "xen/xen.h",
    ])
});

pub static C_STANDARD_HEADER_FOLDERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "sys",
        "arpa",
        "asm-generic",
        "bits",
        "gnu",
        "net",
        "netinet",
        "protocols",
        "rpc",
        "rpcsvc",
        "scsi",
        "drm",
        "linux",
        "misc",
        "mtd",
        "rdma",
        "sound",
        "video",
        "xen",
    ])
});

pub static HEADERS_CONTAINING_TEMPLATES: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("std::vector", vec!["<vector>"]),
            ("std::map", vec!["<map>"]),
            ("std::string", vec!["<string>"]),
            ("std::set", vec!["<set>"]),
            ("std::unordered_map", vec!["<unordered_map>"]),
            ("std::unique_ptr", vec!["<memory>"]),
            ("std::shared_ptr", vec!["<memory>"]),
            ("std::function", vec!["<functional>"]),
            ("std::pair", vec!["<utility>"]),
            ("std::sort", vec!["<algorithm>"]),
            ("std::cout", vec!["<iostream>"]),
            ("std::ostringstream", vec!["<sstream>"]),
            ("std::thread", vec!["<thread>"]),
            ("std::mutex", vec!["<mutex>"]),
            ("std::atomic", vec!["<atomic>"]),
            ("std::future", vec!["<future>"]),
            ("std::optional", vec!["<optional>"]),
            ("std::variant", vec!["<variant>"]),
            ("std::deque", vec!["<deque>"]),
            ("std::queue", vec!["<queue>"]),
            ("std::stack", vec!["<stack>"]),
            ("std::array", vec!["<array>"]),
            ("std::list", vec!["<list>"]),
            ("std::forward_list", vec!["<forward_list>"]),
            ("std::tuple", vec!["<tuple>"]),
            ("std::regex", vec!["<regex>"]),
            ("std::complex", vec!["<complex>"]),
            ("std::valarray", vec!["<valarray>"]),
            ("std::bitset", vec!["<bitset>"]),
            ("std::chrono", vec!["<chrono>"]),
            ("std::filesystem", vec!["<filesystem>"]),
            ("std::span", vec!["<span>"]),
            ("std::any", vec!["<any>"]),
            ("std::numeric_limits", vec!["<limits>"]),
            ("std::runtime_error", vec!["<stdexcept>"]),
            ("std::logic_error", vec!["<stdexcept>"]),
            ("std::streambuf", vec!["<streambuf>"]),
            ("std::ios_base", vec!["<ios>"]),
            ("std::basic_string", vec!["<string>"]),
            ("std::hash", vec!["<functional>"]),
            ("std::greater", vec!["<functional>"]),
            ("std::less", vec!["<functional>"]),
            ("std::equal_to", vec!["<functional>"]),
            ("std::allocator", vec!["<memory>"]),
            ("std::begin", vec!["<iterator>"]),
            ("std::end", vec!["<iterator>"]),
            ("std::move", vec!["<utility>"]),
            ("std::forward", vec!["<utility>"]),
            ("std::make_pair", vec!["<utility>"]),
            ("std::make_shared", vec!["<memory>"]),
            ("std::make_unique", vec!["<memory>"]),
            ("std::min", vec!["<algorithm>"]),
            ("std::max", vec!["<algorithm>"]),
            ("std::find", vec!["<algorithm>"]),
            ("std::count", vec!["<algorithm>"]),
            ("std::swap", vec!["<utility>"]),
            ("std::tie", vec!["<tuple>"]),
            ("std::get", vec!["<tuple>"]),
        ])
    });

pub fn classify_include(
    include_path: &str,
    filename: &str,
    _include_order: &str,
    include_kind: IncludeKind,
) -> u8 {
    let path = std::path::Path::new(include_path);
    let inc_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let inc_ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let file_stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if inc_stem == file_stem {
        return LIKELY_MY_HEADER;
    }

    let inc_first = include_path.split('/').next().unwrap_or("");
    let file_first = filename.split('/').next().unwrap_or("");
    if !inc_first.is_empty() && inc_first == file_first {
        return POSSIBLE_MY_HEADER;
    }

    if !inc_ext.is_empty() {
        if C_HEADERS.contains(include_path) {
            return C_SYS_HEADER;
        }
        let first_seg = include_path.split('/').next().unwrap_or("");
        if C_STANDARD_HEADER_FOLDERS.contains(first_seg) {
            return C_SYS_HEADER;
        }
        // System includes with extensions that aren't known C headers
        if include_kind == IncludeKind::System {
            return OTHER_SYS_HEADER;
        }
    }

    if inc_ext.is_empty() && CPP_HEADERS.contains(include_path) {
        return CPP_SYS_HEADER;
    }

    OTHER_HEADER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpp_headers_contains_vector() {
        assert!(CPP_HEADERS.contains("vector"));
        assert!(CPP_HEADERS.contains("algorithm"));
        assert!(CPP_HEADERS.contains("string"));
    }

    #[test]
    fn test_c_headers_contains_stdio() {
        assert!(C_HEADERS.contains("stdio.h"));
        assert!(C_HEADERS.contains("stdlib.h"));
        assert!(C_HEADERS.contains("string.h"));
    }

    #[test]
    fn test_c_folders_contains_sys() {
        assert!(C_STANDARD_HEADER_FOLDERS.contains("sys"));
        assert!(C_STANDARD_HEADER_FOLDERS.contains("arpa"));
        assert!(C_STANDARD_HEADER_FOLDERS.contains("linux"));
    }

    #[test]
    fn test_classify_likely_my_header() {
        assert_eq!(
            classify_include("browser.h", "browser.cc", "default", IncludeKind::Quoted),
            LIKELY_MY_HEADER
        );
    }

    #[test]
    fn test_classify_c_sys_header() {
        assert_eq!(
            classify_include("stdio.h", "test.cc", "default", IncludeKind::Quoted),
            C_SYS_HEADER
        );
    }

    #[test]
    fn test_classify_cpp_sys_header() {
        assert_eq!(
            classify_include("vector", "test.cc", "default", IncludeKind::Quoted),
            CPP_SYS_HEADER
        );
    }

    #[test]
    fn test_classify_other_header() {
        assert_eq!(
            classify_include("mylib.h", "test.cc", "default", IncludeKind::Quoted),
            OTHER_HEADER
        );
    }

    #[test]
    fn test_classify_possible_my_header() {
        assert_eq!(
            classify_include("chrome/tab.h", "chrome/browser.cc", "default", IncludeKind::Quoted),
            POSSIBLE_MY_HEADER
        );
    }

    #[test]
    fn test_headers_containing_templates() {
        let v = HEADERS_CONTAINING_TEMPLATES.get("std::vector").unwrap();
        assert!(v.contains(&"<vector>"));
    }

    #[test]
    fn test_classify_other_sys_header() {
        // System include (<>) with extension that's not a known C header
        assert_eq!(
            classify_include("boost/optional.hpp", "test.cc", "default", IncludeKind::System),
            OTHER_SYS_HEADER
        );
    }

    #[test]
    fn test_classify_system_include_known_c_header() {
        // Known C header with angle brackets still returns C_SYS_HEADER
        assert_eq!(
            classify_include("stdio.h", "test.cc", "default", IncludeKind::System),
            C_SYS_HEADER
        );
    }
}
