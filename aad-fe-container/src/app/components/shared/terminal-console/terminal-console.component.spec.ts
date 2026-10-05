import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TerminalConsoleComponent } from './terminal-console.component';

describe('TerminalConsoleComponent', () => {
  let component: TerminalConsoleComponent;
  let fixture: ComponentFixture<TerminalConsoleComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TerminalConsoleComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(TerminalConsoleComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
