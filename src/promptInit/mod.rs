use colorize::AnsiColor;
use inquire::{validator::Validation};
use chrono::NaiveDate;




use crate::coinStdinOut::*;

#[allow(unused_variables)]

 fn my_vec_list_crypto()-> Vec<String>{
 /*  let list = vec![
      "bitcoin".to_string(),
      "polkadot".to_string(),
      "cardano".to_string(),
      "etherum".to_string()
  ];
  list
  */                                           //listToken.txt 
   match coinStdinOut::load_token_list("ListToken.txt") {
    Ok(tokens) => {
          if tokens.is_empty() {
            // Si la lista de tokens está vacía, devolver una lista básica.
            vec![
                "bitcoin".to_string(),
                "polkadot".to_string(),
                "cardano".to_string(),
                "ethereum".to_string(),
            ]
        } else {
            tokens
        }
    },
    Err(_) => vec![], // Returns an empty vector if there is an error
}

}

pub fn my_promt_date ()-> NaiveDate{
  let message = "Seleccione fecha de creacion deConsulta.".yellow();
  let msg_error ="Error al seleccionar fecha creacion".red();

  let selected_date = inquire::DateSelect::new(&message)
  .prompt()
  .expect(&msg_error);
selected_date
}
pub fn my_prompt_multiselect()-> Vec<String>{
  let cryptos_list = my_vec_list_crypto();
  let promt_error ="Error al seleccionar los tokens".red();
  let promt_message = "Por favor ingrese una/s cripto/s para buscar".yellow();
  let select = inquire::MultiSelect::new(&promt_message, cryptos_list)
  .with_page_size(10) // Establece el número de elementos visibles por página
  .prompt()
  .expect(&promt_error);

  select
   
}
pub fn my_prompt_select()-> String{
  let cryptos_list = my_vec_list_crypto();

  let promt_error ="Error al seleccionar un token".red();
  let promt_message = "Por favor ingrese una cripto para buscar".yellow();
  let select = inquire::Select::new(&promt_message,cryptos_list)
  .prompt()
  .expect(&promt_error);
select

}
pub fn my_prompt_text()-> String{
  let prompt_message = "Nombre de la consulta ...".yellow();
  let validator = |input : &str| if input.chars().count() < 20 {
      let f_chart = input.chars().next().unwrap() as u8 ;
      match f_chart{
          65..=90 => {
              return Ok(Validation::Valid);
          }
           _     => {
              return Ok(Validation::Invalid("Nombre debe empezar con Mayuscula".into()));
          }    
           }
      }else {
          return Ok(Validation::Invalid("Nombre muy largo , solo puede usar 20 caracteres".into()));

      };

  
  let prompt_name = inquire::Text::new(&prompt_message)
       .with_validator(validator)
       .prompt()
       .expect("Error al obtener nombre consulta .");
  
  prompt_name 
}
pub fn my_prompt_boolean()-> bool {
  let message = "Desea configurar/ modificar  el listado de tokens a listar ?".yellow();
  let msg_error = "Error  to proceed?".red();
  let proceed = inquire::prompt_confirmation(message);
  //.expect(&msg_error);
  
  if proceed.is_err(){
      println!("{0}", msg_error);
      return false;
  }
  
 proceed.unwrap()

}