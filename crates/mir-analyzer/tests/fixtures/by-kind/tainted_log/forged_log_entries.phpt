===description===
A request value in a log message can contain a line break that forges entries.
===config===
<mir>
  <issueHandlers>
    <TaintedLog errorLevel="error"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Psr\Log {
    interface LoggerInterface {
        public function warning(string|\Stringable $message, array $context = []): void;
        public function log(mixed $level, string|\Stringable $message, array $context = []): void;
    }
}

namespace App {
    use Psr\Log\LoggerInterface;

    function login(LoggerInterface $logger): void {
        error_log('Login attempt: ' . $_POST['user']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLog: Tainted log message — possible log injection
        $logger->warning('Failed login for ' . $_POST['user']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLog: Tainted log message — possible log injection
        $logger->log('info', "User {$_GET['name']} logged out");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLog: Tainted log message — possible log injection
    }
}
