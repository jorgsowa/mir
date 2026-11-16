===description===
Messages whose line breaks are removed or escaped can't forge a log entry;
removing only the line feed leaves the carriage return.
===config===
<mir>
  <issueHandlers>
    <TaintedLog errorLevel="error"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <PossiblyInvalidOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function log_user(): void {
    $user = $_GET['user'];
    error_log('login ' . str_replace(["\r", "\n"], '', $user));
    error_log('login ' . str_replace(["\r\n", "\n", "\r"], ' ', $user));
    error_log('login ' . preg_replace('/[\r\n]+/', ' ', $user));
    error_log('login ' . preg_replace('/\s+/', ' ', $user));
    error_log('login ' . json_encode($user));
    error_log('login ' . strtr($user, ["\r" => '', "\n" => '']));
    error_log('login ' . addcslashes($user, "\r\n"));
    error_log('login ' . str_replace("\n", '', $user));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLog: Tainted log message — possible log injection
}
