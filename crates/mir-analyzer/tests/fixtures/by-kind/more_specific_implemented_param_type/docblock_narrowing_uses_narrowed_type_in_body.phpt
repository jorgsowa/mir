===description===
When an override narrows the @param type via docblock, the method body may
call methods available only on the narrowed subtype without triggering
UndefinedMethod — the analyzer uses the docblock type for intra-body checks.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Message {}
class EmailMessage extends Message {
    public function getSubject(): string { return ''; }
}

class Processor {
    public function process(Message $msg): void {}
}

class EmailProcessor extends Processor {
    /** @param EmailMessage $msg */
    public function process(Message $msg): void {
        $subject = $msg->getSubject();
    }
}
===expect===
