===description===
foreach value used only as an argument to a dynamic method call is not reported as unused
===file===
<?php
class Mailer {
    /** @param array<string> $addresses */
    public function to(array $addresses): void {}
//                     ^^^^^^^^^^^^^^^^ UnusedParam: Parameter $addresses is never used
    /** @param array<string> $addresses */
    public function cc(array $addresses): void {}
//                     ^^^^^^^^^^^^^^^^ UnusedParam: Parameter $addresses is never used
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
