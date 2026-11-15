===description===
Values passed in the PSR-3 context array and request data without line breaks
don't forge entries.
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
    }
}

namespace App {
    use Psr\Log\LoggerInterface;

    function login(LoggerInterface $logger): void {
        $logger->warning('Failed login for {user}', ['user' => $_POST['user']]);
        error_log('Login attempt for user #' . (int) $_POST['id']);
        $name = (string) $_GET['name'];
        if (ctype_alnum($name)) {
            error_log('User ' . $name);
        }
    }
}
