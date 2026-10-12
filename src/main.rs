// Lezione 00: il tuo primo programma Rust.
//
// `fn main()` è il punto di ingresso: quando lanci `cargo run`, Rust
// esegue questa funzione. Più avanti qui dentro vivrà il game loop.

fn main() {
    // `let` crea una variabile. Di default è immutabile: non puoi cambiarla.
    let titolo = "EGRESSO";

    // `let mut` la rende mutabile.
    let mut tentativi_di_uscita = 0;
    tentativi_di_uscita += 1;

    // `println!` stampa una riga. Il `!` indica che è una *macro*, non una
    // funzione normale. `{}` viene sostituito dai valori che seguono.
    println!("{titolo}: tentativo di uscita n. {}", tentativi_di_uscita);
    println!("L'uscita non c'è. Prova a chiudere gli occhi.");

    // ESERCIZIO 00 (vedi docs/diario/00-setup.md): modifica questo programma.
}
