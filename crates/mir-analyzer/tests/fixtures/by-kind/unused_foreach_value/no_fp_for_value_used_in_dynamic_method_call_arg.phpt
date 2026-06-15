===description===
foreach value used only as an argument to a dynamic method call is not reported as unused
===file===
<?php
class Mailer {
    /** @param array<string> $addresses */
    public function to(array $addresses): void {}
    /** @param array<string> $addresses */
    public function cc(array $addresses): void {}
}

function sendMail(Mailer $mailer): void {
    $recipients = [['a@example.com', 'b@example.com'], ['c@example.com']];
    $methods = ['to', 'cc'];
    foreach ($methods as $idx => $method) {
        foreach ($recipients[$idx] as $address) {
            $mailer->{$method}([$address]);
        }
    }
}
===expect===
UnusedParam@4:23-4:39: Parameter $addresses is never used
UnusedParam@6:23-6:39: Parameter $addresses is never used
