fn calculate_invoice_total(items: Vec<Item>, tax_rate: f64, discount: Option<f64>) -> Result<f64, String> {
    if items.is_empty() {
        return Err("No items provided".to_string());
    }

    let mut subtotal = 0.0;
    for item in &items {
        if item.quantity <= 0 {
            return Err(format!("Invalid quantity for item {}", item.name));
        }
        if item.price < 0.0 {
            return Err(format!("Invalid price for item {}", item.name));
        }
        subtotal += item.quantity as f64 * item.price;
    }

    let discount_amount = match discount {
        Some(d) if d > 0.0 && d <= 1.0 => subtotal * d,
        Some(d) if d > 1.0 => d,
        Some(_) => return Err("Invalid discount".to_string()),
        None => 0.0,
    };

    let taxable_amount = subtotal - discount_amount;
    if taxable_amount < 0.0 {
        return Err("Discount cannot exceed subtotal".to_string());
    }

    let tax_amount = taxable_amount * tax_rate;
    if tax_rate < 0.0 || tax_rate > 1.0 {
        return Err("Invalid tax rate".to_string());
    }

    let total = taxable_amount + tax_amount;

    // Round to 2 decimal places
    let rounded_total = (total * 100.0).round() / 100.0;

    Ok(rounded_total)
}

struct Item {
    name: String,
    quantity: i32,
    price: f64,
}
