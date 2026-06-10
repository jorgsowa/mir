===description===
Tainted llm prompt from user input
===file===
<?php
class LlmAgent {
    /** @taint-sink llm_prompt $prompt */
    public function prompt(string $prompt): string {
        return "";
    }
}

$agent = new LlmAgent();
$agent->prompt((string) $_GET["question"]);
===expect===
