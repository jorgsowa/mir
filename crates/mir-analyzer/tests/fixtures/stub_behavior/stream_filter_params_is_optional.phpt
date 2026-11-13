===description===
stream_filter_append() and stream_filter_prepend() take $params by name without $mode
===file===
<?php
/** @param resource $s */
function f($s): void {
    stream_filter_append($s, 'string.rot13', params: ['a' => 1]);
    stream_filter_prepend($s, 'string.rot13', params: []);
    stream_filter_append($s, 'string.rot13');
}
