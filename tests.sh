examples=(canvas hello hello_app hello_load mouse_pick ticks zoom_rotator widgets)

for val in ${examples[@]}; do
    cargo run --release --bin ${val}
    wait $!
done
