===description===
A prefix pattern on another class, here an imported interface, limits the mask to the matching constants.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib {
    interface Logger {
        const LEVEL_INFO = 1;
        const LEVEL_WARN = 2;
        const CHANNEL_MAIL = 64;
    }
}

namespace App {
    use Lib\Logger;

    final class Reporter {
        /** @param int-mask-of<Logger::LEVEL_*> $levels */
        public function listen(int $levels): void {}
    }

    function run(Reporter $reporter): void {
        $reporter->listen(Logger::LEVEL_INFO | Logger::LEVEL_WARN);
        $reporter->listen(3);
        $reporter->listen(Logger::CHANNEL_MAIL);
//                        ^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $levels of listen() expects '0|1|2|3', got '64'
        $reporter->listen(4);
//                        ^ InvalidArgument: Argument $levels of listen() expects '0|1|2|3', got '4'
    }
}
