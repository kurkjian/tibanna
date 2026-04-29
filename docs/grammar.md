# Tibanna Grammar

$$
\begin{align}
\text{program} &\to \text{[stmt]}^+ \\
\text{stmt} &\to
    \begin{cases}
        exit(\text{[expr]}); \\
        let \space \text{[ident]} = \text{[expr]}; \\
        if \space \text{[expr]} \space \{\text{[scope]}\} \space \text{[else\_clause]} \\
        while \space \text{[expr]} \space \{\text{[scope]}\} \\
        \text{[ident]} = \text{[expr]}; \\
        \{\text{[scope]}\} \\
    \end{cases} \\
\text{expr} &\to
    \begin{cases}
        \text{[bin\_expr]} \\
        \text{[term]} \\
    \end{cases} \\
\text{bin\_expr} &\to
    \begin{cases}
        \text{[expr] + [expr]} \\
        \text{[expr] - [expr]} \\
        \text{[expr]} \cdot \text{[expr]} \\
        \text{[expr]} < \text{[expr]} \\
        \text{[expr]} <= \text{[expr]} \\
        \text{[expr]} > \text{[expr]} \\
        \text{[expr]} >= \text{[expr]} \\
        \text{[expr]} == \text{[expr]} \\
        \text{[expr]} \space \text{!=} \space \text{[expr]} \\
        \text{[expr]} \space \&\& \space \text{[expr]} \\
        \text{[expr]} \space || \space \text{[expr]} \\
    \end{cases} \\
\text{scope} &\to \text{[stmt]}^* \\
\text{else\_clause} &\to
    \begin{cases}
        else \space if \space \text{[expr]} \space \{\text{[scope]}\} \space \text{[else\_clause]} \\
        else \space \{\text{[scope]}\} \\
        \epsilon
    \end{cases} \\
\text{term} &\to
    \begin{cases}
        \text{ident} \\
        \text{intlit} \\
        \text{bool} \\
    \end{cases} \\
\text{ident} &\to \text{[a-Z]}^+\text{[a-Z0-9 | \_]}^* \\
\text{intlit} &\to \mathbb{Z} \\
\text{bool} &\to
    \begin{cases}
        true \\
        false \\
    \end{cases}
\end{align} \\
$$
