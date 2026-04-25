# Tibanna Grammar

$$
\begin{align}
\text{program} &\to \text{[stmt]}^+ \\
\text{stmt} &\to
    \begin{cases}
        exit(\text{[expr]}); \\
        let \space \text{[ident]} = \text{[expr]}; \\
        if \space \text{[expr]} \space \{[scope]\} \\
        \text{[ident]} = \text{[expr]}; \\
        \{[scope]\} \\
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
    \end{cases} \\
\text{scope} &\to \text{[stmt]}^* \\
\text{term} &\to
    \begin{cases}
        \text{ident} \\
        \text{intlit} \\
    \end{cases} \\
\text{ident} &\to \text{[a-Z]}^+\text{[a-Z0-9 | \_]}^* \\
\text{intlit} &\to \mathbb{Z}
\end{align}
$$
