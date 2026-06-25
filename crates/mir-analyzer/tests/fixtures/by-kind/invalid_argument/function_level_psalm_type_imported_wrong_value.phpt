===description===
Passing a wrong value when the param type comes from a @psalm-import-type alias triggers InvalidArgument
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

/**
 * @psalm-type Level = "debug"|"info"|"warning"|"error"
 */
class Logger {}

/**
 * @psalm-import-type Level from Logger
 * @param Level $level
 */
function log(string $level): void {}

log("trace");
===expect===
InvalidArgument@15:4-15:11: Argument $level of log() expects '"debug"|"info"|"warning"|"error"', got '"trace"'
