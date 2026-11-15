===description===
A request value passed as an argument to a program run without a shell can start
with `-` and turn into an option, such as `--upload-pack=...` for `git clone`.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\Process {
    class Process {
        /** @param list<string> $command */
        public function __construct(array $command) {}
    }
}

namespace App {
    use Symfony\Component\Process\Process;

    function cloneRepo(): void {
        $spec = [];
        proc_open(['git', 'clone', $_GET['repo']], $spec, $pipes);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCommandArgument: Tainted command argument can start with '-' — possible argument injection
    }

    function connect(): void {
        $host = $_POST['host'];
        new Process(['ssh', $host, 'uptime']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCommandArgument: Tainted command argument can start with '-' — possible argument injection
    }

    function checkedTooLate(): void {
        $file = $_GET['file'];
        new Process(['grep', '-r', $file, '--', '.']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCommandArgument: Tainted command argument can start with '-' — possible argument injection
    }
}
