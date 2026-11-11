===description===
A `//` inside a quoted literal of a shape is kept; only a trailing comment is dropped.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/**
 * @param array{
 *   scheme: 'http://'|'https://', // with the separator
 *   host: string,
 * } $url
 */
function open(array $url): void {}

open(['scheme' => 'https://', 'host' => 'example.org']);
open(['scheme' => 'ftp://', 'host' => 'example.org']);
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $url of open() expects 'array{'scheme': "http://"|"https://", 'host': string}', got 'array{'scheme': "ftp://", 'host': "example.org"}'
