===description===
Literal string as int array key
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
class a {
    private const REDIRECTS = [
        "a" => [
            "from" => "79268724911",
            "to" => "74950235931",
        ],
        "b" => [
            "from" => "79313044964",
            "to" => "78124169167",
        ],
    ];

    private const SIP_FORMAT = "sip:%s@voip.test.com:9090";

    /** @return array<string, string> */
    public function test(): array {
        $redirects = [];
        foreach (self::REDIRECTS as $redirect) {
            $redirects[$redirect["from"]] = sprintf(self::SIP_FORMAT, $redirect["to"]);
        }

        return $redirects;
    }
}
===expect===
